===description===
`$this->prop === Foo::class` (plain class-string, not an enum case) proves
the receiver non-null too — sibling branches (enum-case, get_class(),
get_debug_type()) already excluded null on the receiver this way; the
plain class-string branch didn't.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <MissingPropertyType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Foo {}

final class Widget {
    /** @var class-string<Foo> */
    public $type = Foo::class;
    public function realMethod(): void {}
}

function narrows(?Widget $w): void {
    if ($w->type === Foo::class) {
//      ^^^^^^^^ PossiblyNullPropertyFetch: Cannot access property $type on possibly null value
        $w->realMethod();
    }
}

function narrowsSymmetric(?Widget $w): void {
    if (Foo::class === $w->type) {
//                     ^^^^^^^^ PossiblyNullPropertyFetch: Cannot access property $type on possibly null value
        $w->realMethod();
    }
}

function stillFlaggedWithoutMatch(?Widget $w): void {
    $w->realMethod();
//  ^^^^^^^^^^^^^^^^ PossiblyNullMethodCall: Cannot call method realMethod() on possibly null value
}
