===description===
Passing a static property by reference to a built-in function
(`array_push(Bag::$queue, 1)`) mutates it exactly as much as
`Bag::$queue = ...` would, but `check_byref_arg_purity` had no
`StaticPropertyAccess` arm at all.
===config===
suppress=ImpureFunctionCall,MixedArgument
===file===
<?php
class Bag {
    public static array $queue = [];
}

/** @pure */
function enqueue(): void {
    array_push(Bag::$queue, 1);
}
===expect===
ImpureStaticPropertyAssignment@8:15-8:26: Assigning to static property Bag::$queue in a @pure function
ImpureStaticPropertyAccess@8:20-8:26: Reading static property Bag::$queue in a @pure function
