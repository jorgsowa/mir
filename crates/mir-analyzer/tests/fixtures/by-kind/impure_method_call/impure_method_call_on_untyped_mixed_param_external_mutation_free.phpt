===description===
Calling a method on an untyped (mixed) parameter inside a
@psalm-external-mutation-free method was never flagged — the blanket
unresolvable-receiver check only ever gated on `is_in_pure_fn`, unlike
the resolved-callee checks a few lines below it which also cover
`is_in_external_mutation_free_method` for the same parameter-receiver shape.
===config===
suppress=MissingParamType,MixedMethodCall,MissingReturnType
===file===
<?php
class Runner {
    /** @psalm-external-mutation-free */
    public function run($a): void {
        $a->mutate();
    }
}
===expect===
ImpureMethodCall@5:8-5:20: Calling impure method mutate() in a pure or immutable context
