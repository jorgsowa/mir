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
PossiblyNullArgument
===ignore===
TODO
