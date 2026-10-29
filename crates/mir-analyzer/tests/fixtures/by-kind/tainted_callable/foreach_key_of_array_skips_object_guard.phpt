===description===
The key of an iterated array is an int or a string, so request data in it never passes a callee's `instanceof` guard; called directly it is still a tainted callable.
===file===
<?php
class Query {
    public function where(mixed $column): void {
        if ($column instanceof Closure) {
            $column($this);
        }
    }
}

function guarded(array $credentials, Query $q): void {
    foreach ($credentials as $key => $value) {
//                                   ^^^^^^ MixedAssignment: Variable $value is assigned a mixed type
//                                   ^^^^^^ UnusedForeachValue: Foreach value $value is never read
        $q->where($key);
    }
}

function direct(array $credentials): void {
    foreach ($credentials as $key => $value) {
//                                   ^^^^^^ MixedAssignment: Variable $value is assigned a mixed type
//                                   ^^^^^^ UnusedForeachValue: Foreach value $value is never read
        $key();
//      ^^^^^^ MixedFunctionCall: Cannot call mixed type as a function
    }
}

function callers(Query $q): void {
    $token = (string) $_GET['t'];
//                    ^^^^^^^^^^ MixedArrayAccess: Array access on mixed type
    guarded([$token => 1], $q);
    direct([$token => 1]);
//  ^^^^^^^^^^^^^^^^^^^^^ TaintedCallable: Tainted callable — user input chooses the code that runs
}
