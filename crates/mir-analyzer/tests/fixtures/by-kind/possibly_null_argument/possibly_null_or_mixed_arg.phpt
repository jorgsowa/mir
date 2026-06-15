===description===
Possibly null or mixed arg
===file===
<?php
class A {
    /**
     * @var mixed
     */
    public $foo;
}

function takesString(string $s) : void {}

function takesA(?A $a) : void {
    /**
     * @suppress PossiblyNullPropertyFetch
     * @suppress MixedArgument
     */
    takesString($a->foo);
}
===expect===
MissingPropertyType@6:4-6:15: Property A::$foo has no type annotation
UnusedParam@9:21-9:30: Parameter $s is never used
