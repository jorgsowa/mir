===description===
`class_exists($obj->prop)` narrows the receiver non-null in the true branch —
class_exists(null) can never be true, so a true result also proves $obj
itself wasn't null. The property was already narrowed to class-string but
the receiver var was left untouched.
===config===
suppress=UnusedVariable,MissingPropertyType
===file===
<?php
class Foo {
    /** @var string */
    public $className;
    public function ping(): void {}
}

function narrowsReceiver(?Foo $obj): void {
    if (class_exists($obj->className)) {
//                   ^^^^^^^^^^^^^^^ PossiblyNullArgument: Argument $class of class_exists() might be null
//                   ^^^^^^^^^^^^^^^ PossiblyNullPropertyFetch: Cannot access property $className on possibly null value
        $obj->ping();
    }
}

function doesNotNarrowOutsideBranch(?Foo $obj): void {
    if (class_exists($obj->className)) {
//                   ^^^^^^^^^^^^^^^ PossiblyNullArgument: Argument $class of class_exists() might be null
//                   ^^^^^^^^^^^^^^^ PossiblyNullPropertyFetch: Cannot access property $className on possibly null value
        $_ = 1;
    }
    $obj->ping();
//  ^^^^^^^^^^^^ PossiblyNullMethodCall: Cannot call method ping() on possibly null value
}
===expect===
