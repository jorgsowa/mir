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
MissingConstructor@2:0-2:9: Class A has uninitialized properties but no constructor
UnusedPsalmSuppress@16:0-16:0: Suppress annotation for 'MixedArgument' is never used
