use state_lens::{
    analysis::Engine,
    model::{AnnotationLabel as L, AnnotationTarget as T, DocumentReport},
};
fn report(language: &str, source: &str) -> DocumentReport {
    let mut engine = Engine::new().unwrap();
    engine
        .set("file:///sample".into(), language, source.into(), Some(1))
        .unwrap();
    engine.analyze().documents.remove(0)
}
#[test]
fn explicit_async_is_not_generic_impurity() {
    for (language, source) in [
        ("typescript", "async function load() { return 1; }"),
        ("javascript", "async function load() { return 1; }"),
        ("python", "async def load():\n    return 1\n"),
        ("rust", "async fn load() -> i32 { 1 }"),
        ("go", "package main\nfunc main(){ go func(){}() }"),
    ] {
        let document = report(language, source);
        assert!(!document.parse_errors, "{language}");
        assert!(
            document
                .annotations
                .iter()
                .any(|a| a.label == L::Async && a.target == T::Function),
            "{language}"
        );
    }
    let document = report("typescript", "function load(){ return 'async'; }");
    assert!(!document.annotations.iter().any(|a| a.label == L::Async));
}
#[test]
fn mutation_targets_and_functions_are_distinct() {
    for (language, source) in [
        ("typescript", "function f(){ let x=1; x+=1; return x; }"),
        ("javascript", "function f(){ let x=1; x+=1; return x; }"),
        ("rust", "fn f(){let mut x=1; x+=1;}"),
        ("python", "def f():\n    x=1\n    x+=1\n"),
        ("go", "package main\nfunc f(){x:=1; x+=1}"),
    ] {
        let document = report(language, source);
        for (label, target) in [
            (L::Impure, T::Function),
            (L::Volatile, T::Property),
            (L::Mutacious, T::Segment),
        ] {
            assert!(
                document
                    .annotations
                    .iter()
                    .any(|a| a.label == label && a.target == target),
                "{language} {label:?}"
            );
        }
    }
}
#[test]
fn external_inputs_require_supported_unshadowed_sources() {
    for (language, source) in [
        ("typescript", "async function f(){const data=await fetch('/api'); return data;}"),
        ("javascript", "async function f(){const data=await fetch('/api'); return data;}"),
        ("python", "def f():\n    data=input()\n    return data\n"),
        ("go", "package main\nimport \"net/http\"\nfunc f(){data,err:=http.Get(\"url\"); _,_=data,err}"),
        ("rust", "fn f(){let data=std::env::args();}"),
    ] { assert!(report(language,source).annotations.iter().any(|a| a.label==L::Untyped && a.target==T::Property),"{language}"); }
    let shadow = report(
        "typescript",
        "function f(fetch){ const data=fetch(); return data; }",
    );
    assert!(!shadow
        .annotations
        .iter()
        .any(|a| a.label == L::Unstructured));
}
#[test]
fn only_supported_pairs_receive_lifecycle_evidence() {
    for (language, source) in [
        ("typescript", "import * as fs from 'node:fs'; function f(){const file=fs.openSync('x','r'); fs.closeSync(file);}"),
        ("javascript", "import * as fs from 'node:fs'; function f(){const file=fs.openSync('x','r'); fs.closeSync(file);}"),
        ("python", "def f():\n    file=open('x')\n    file.close()\n"),
        ("go", "package main\nimport \"os\"\nfunc f(){file,err:=os.Open(\"x\"); _=err; defer file.Close()}"),
        ("rust", "fn f(){let file=std::fs::File::open(\"x\"); drop(file);}"),
    ] { assert!(report(language, source).findings.iter().any(|a| a.resolution=="resource-lifecycle"), "{language}"); }
    for source in ["function f(){const file=open(); file.close();}", "import * as fs from 'node:fs'; function f(){let file=fs.openSync('x','r'); file=other; fs.closeSync(file);}"] {
        assert!(!report("javascript",source).findings.iter().any(|a| a.resolution=="resource-lifecycle"));
    }
}
#[test]
fn specific_labels_require_specific_evidence() {
    let document=report("python","def f():\n    data=input()\n    if isinstance(data, str):\n        eval(data)\n    else:\n        print(data)\n");
    assert!(document
        .annotations
        .iter()
        .any(|a| a.label == L::Polymorphic));
    assert!(document
        .findings
        .iter()
        .any(|a| a.resolution == "unsafe-input-execution"));
    let execution = document
        .findings
        .iter()
        .find(|f| f.resolution == "unsafe-input-execution")
        .unwrap();
    assert_eq!(execution.evidence.len(), 2);
    assert_ne!(execution.evidence[0].range, execution.evidence[1].range);
    let safe = report(
        "python",
        "def f():\n    data=input()\n    data='constant'\n    eval(data)\n",
    );
    assert!(!safe
        .findings
        .iter()
        .any(|a| a.resolution == "unsafe-input-execution"));
    let division = report("rust", "fn f(){ let x=0; let y=1/x; }");
    assert!(!division
        .annotations
        .iter()
        .any(|a| matches!(a.label, L::Polymorphic | L::Insecure)));
}
#[test]
fn edits_remove_annotations_and_preserve_evidence_indices() {
    let mut engine = Engine::new().unwrap();
    engine
        .set(
            "file:///sample.js".into(),
            "javascript",
            "async function f(){let x=1;x++;return fetch('/api');}".into(),
            Some(1),
        )
        .unwrap();
    for document in &engine.analyze().documents {
        for a in &document.annotations {
            assert!(a.findings.iter().all(|&i| i < document.findings.len()));
        }
    }
    engine
        .set(
            "file:///sample.js".into(),
            "javascript",
            "function f(){return 1;}".into(),
            Some(2),
        )
        .unwrap();
    assert!(engine.analyze().documents[0].annotations.is_empty());
}

#[test]
fn type_dispatch_and_execution_follow_the_input_binding() {
    for language in ["typescript", "javascript"] {
        let document = report(language, "async function f(){const response=await fetch('/api'); const data=await response.text(); if(typeof data === 'string'){eval(data);}else{return data;}}");
        assert!(
            document
                .annotations
                .iter()
                .any(|a| a.label == L::Polymorphic),
            "{language}"
        );
        assert!(
            document
                .findings
                .iter()
                .any(|a| a.resolution == "unsafe-input-execution"),
            "{language}"
        );
        for source in [
            "async function f(){const data=await fetch('/api'); eval(data);}",
            "async function f(){const response=await fetch('/api'); const data=await response.text(); exec(data);}",
            "async function f(){const data=await fetch('/api'); const x=1; if(typeof x === 'number'){return 1;}else{return data;}}",
            "async function f(){const data=validate(await fetch('/api')); if(typeof data === 'string'){return 1;}else{return data;}}",
            "async function f(){let response=await fetch('/api'); response=local; const data=await response.text(); eval(data);}",
        ] {
            assert!(!report(language, source).annotations.iter().any(|a| matches!(a.label, L::Polymorphic | L::Insecure)), "{source}");
        }
    }
    for source in [
        "def f():\n    data=validate(input())\n    eval(data)\n",
        "def f():\n    data=input()\n    eval('1', data)\n",
        "def f():\n    data=input()\n    data=validate(data)\n    eval(data)\n",
        "def f():\n    data=input()\n    def nested(data):\n        eval(data)\n",
        "def f():\n    data=input()\n    x=1\n    if isinstance(x, str):\n        return x\n    else:\n        return data\n",
    ] {
        assert!(!report("python", source).annotations.iter().any(|a| matches!(a.label, L::Polymorphic | L::Insecure)), "{source}");
    }
}

#[test]
fn resource_pairs_do_not_cross_lexical_scopes_or_rebindings() {
    for source in [
        "import * as fs from 'fs'; function f(){{const file=fs.openSync('x','r');} fs.closeSync(file);}",
        "import * as fs from 'fs'; function f(){const file=fs.openSync('x','r'); function g(){fs.closeSync(file);}}",
        "import * as fs from 'fs'; function f(fs){const file=fs.openSync('x','r'); fs.closeSync(file);}",
        "import * as fs from 'fs'; function f(){const file=wrap(fs.openSync('x','r')); fs.closeSync(file);}",
    ] {
        assert!(!report("javascript", source).findings.iter().any(|a| a.resolution == "resource-lifecycle"), "{source}");
    }
    assert!(!report("go", "package main\nimport \"os\"\nfunc f(){file,err:=os.Open(\"x\"); file,err=other(); defer file.Close(); _=err}").findings.iter().any(|a| a.resolution == "resource-lifecycle"));
}

#[test]
fn imported_and_local_api_lookalikes_are_not_builtin_sources() {
    for (language, source) in [
        (
            "python",
            "import input\ndef f():\n    data=input()\n    eval(data)\n",
        ),
        (
            "python",
            "def imports():\n    import requests\ndef f():\n    data=requests.get('url')\n",
        ),
        (
            "javascript",
            "import {fetch} from './local'; function f(){const data=fetch();}",
        ),
        (
            "python",
            "from custom import input\ndef f():\n    data=input()\n    eval(data)\n",
        ),
        (
            "python",
            "from custom import open\ndef f():\n    file=open('x')\n    file.close()\n",
        ),
        (
            "rust",
            "mod reqwest {pub fn get() {}} fn f(){let data=reqwest::get();}",
        ),
    ] {
        assert!(
            !report(language, source)
                .annotations
                .iter()
                .any(|a| matches!(a.label, L::Unstructured | L::Insecure | L::Lifecycle)),
            "{source}"
        );
    }
}

#[test]
fn disabled_checks_invalidate_cached_reports_and_dependent_compounds() {
    let mut engine = Engine::new().unwrap();
    engine
        .set(
            "file:///sample.py".into(),
            "python",
            "def f():\n    count=2\n    count-=2\n    return 10/count\n".into(),
            Some(1),
        )
        .unwrap();
    let initial = engine.analyze();
    let compound = initial.documents[0]
        .findings
        .iter()
        .find(|f| f.compound.is_some())
        .expect("linked mutable denominator");
    assert!(!compound.blast_radius.is_empty());
    let ingredient = compound.compound.as_ref().unwrap().ingredients[0]
        .rule_id
        .split('.')
        .next_back()
        .unwrap()
        .to_string();
    engine.set_disabled_checks([ingredient.clone()].into_iter().collect());
    let disabled = engine.analyze();
    assert!(disabled.generation > initial.generation);
    assert!(disabled.documents[0]
        .findings
        .iter()
        .all(|f| f.defect != ingredient && f.compound.is_none()));
    assert!(disabled.documents[0].annotations.iter().all(|a| a
        .findings
        .iter()
        .all(|i| *i < disabled.documents[0].findings.len())));
    engine.set_disabled_checks(Default::default());
    assert!(engine.analyze().documents[0]
        .findings
        .iter()
        .any(|f| f.compound.is_some()));
}

#[test]
fn unrelated_mutation_and_guarded_division_do_not_compound() {
    for source in [
        "def f(count):\n    other=2\n    other-=2\n    return 10/count\n",
        "def f():\n    count=2\n    count-=2\n    if count != 0:\n        return 10/count\n",
    ] {
        assert!(report("python", source)
            .findings
            .iter()
            .all(|f| f.compound.is_none()));
    }
}

#[test]
fn compound_radius_is_the_division_expression_with_unicode_before_it() {
    let source = "def f():\n    marker='💥'\n    count=2\n    count-=2\n    return 10/count\n";
    let document = report("python", source);
    let compound = document
        .findings
        .iter()
        .find(|f| f.compound.is_some())
        .unwrap();
    assert_eq!(
        &source[compound.trigger.start..compound.trigger.end],
        "count"
    );
    assert_eq!(compound.blast_radius.len(), 1);
    let radius = compound.blast_radius[0];
    assert_eq!(&source[radius.start..radius.end], "10/count");
    let ingredients = &compound.compound.as_ref().unwrap().ingredients;
    assert_ne!(ingredients[0].range, ingredients[1].range);
    for ingredient in ingredients {
        assert!(document
            .findings
            .iter()
            .any(|f| f.rule_id == ingredient.rule_id && f.trigger == ingredient.range));
    }
}
