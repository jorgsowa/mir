===description===
Only the bare (unbound) instantiation is lenient: an inferred mismatching
binding or a subclass that fixes the ancestor's template still errors.
===config===
suppress=UnusedVariable,UnusedParam,MissingConstructor
===file===
<?php
/** @template T */
class Gen {
    /** @param T $v */
    public function __construct(public mixed $v = null) {}
}

/** @extends Gen<string> */
class StrGen extends Gen {}

class Holder {
    /** @var Gen<int> */
    public Gen $a;
    /** @var Gen<int> */
    public Gen $b;

    public function __construct() {
        $this->a = new Gen('s');
        $this->b = new StrGen();
    }
}

/** @param Gen<int> $g */
function take(Gen $g): void {}

function caller(): void {
    take(new StrGen());
}
===expect===
InvalidPropertyAssignment@18:8-18:31: Property $a expects 'Gen<int>', cannot assign 'Gen<string>'
InvalidPropertyAssignment@19:8-19:31: Property $b expects 'Gen<int>', cannot assign 'StrGen<string>'
InvalidArgument@27:9-27:21: Argument $g of take() expects 'Gen<int>', got 'StrGen<string>'
