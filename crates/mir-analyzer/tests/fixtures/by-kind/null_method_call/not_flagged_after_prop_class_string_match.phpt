===description===
`$this->prop === Foo::class` (plain class-string, not an enum case) proves
the receiver non-null too — sibling branches (enum-case, get_class(),
get_debug_type()) already excluded null on the receiver this way; the
plain class-string branch didn't.
===config===
suppress=UnusedParam,MissingPropertyType
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
        $w->realMethod();
    }
}

function narrowsSymmetric(?Widget $w): void {
    if (Foo::class === $w->type) {
        $w->realMethod();
    }
}

function stillFlaggedWithoutMatch(?Widget $w): void {
    $w->realMethod();
}
===expect===
PossiblyNullPropertyFetch@11:8-11:16: Cannot access property $type on possibly null value
PossiblyNullPropertyFetch@17:23-17:31: Cannot access property $type on possibly null value
PossiblyNullMethodCall@23:4-23:20: Cannot call method realMethod() on possibly null value
