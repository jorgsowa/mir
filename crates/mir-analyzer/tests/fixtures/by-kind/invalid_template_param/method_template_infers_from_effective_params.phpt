===description===
Method-level `@template` inference reuses the substituted
`effective_params` (like static_call.rs already does), not raw params —
otherwise a `T|TDefault`-shaped method param binds the whole argument to
the method's own template directly, skipping the class-template
exemption and producing a false `InvalidTemplateParam`.
===config===
suppress=UnusedVariable,MissingConstructor,UnusedParam
===file===
<?php
/** @template T */
class Box {
    /** @param T $item */
    public function __construct($item) {}

    /**
     * @template TDefault of Countable
     * @param T|TDefault $default
     * @return T|TDefault
     */
    public function getOr($default) {
        return $default;
    }
}

class Foo {}

function test(): void {
    $box = new Box(new Foo());
    $box->getOr(new Foo());
}
===expect===
