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
        $obj->ping();
    }
}

function doesNotNarrowOutsideBranch(?Foo $obj): void {
    if (class_exists($obj->className)) {
        $_ = 1;
    }
    $obj->ping();
}
===expect===
PossiblyNullArgument@9:21-9:36: Argument $class of class_exists() might be null
PossiblyNullPropertyFetch@9:21-9:36: Cannot access property $className on possibly null value
PossiblyNullArgument@15:21-15:36: Argument $class of class_exists() might be null
PossiblyNullPropertyFetch@15:21-15:36: Cannot access property $className on possibly null value
PossiblyNullMethodCall@18:4-18:16: Cannot call method ping() on possibly null value
