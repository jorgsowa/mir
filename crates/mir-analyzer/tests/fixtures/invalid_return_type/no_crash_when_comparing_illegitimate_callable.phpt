===description===
noCrashWhenComparingIllegitimateCallable
===file===
<?php
class C {}

function foo() : C {
    return fn (int $i) => "";
}
===expect===
InvalidReturnStatement
===ignore===
TODO
