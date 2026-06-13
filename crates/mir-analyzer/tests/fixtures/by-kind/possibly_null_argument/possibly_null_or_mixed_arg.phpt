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
MissingPropertyType@6:5-6:16: Property A::$foo has no type annotation
UnusedParam@9:22-9:31: Parameter $s is never used
