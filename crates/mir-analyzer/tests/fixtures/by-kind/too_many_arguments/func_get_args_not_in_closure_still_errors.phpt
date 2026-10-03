===description===
func_get_args() inside a closure body does not suppress TooManyArguments for the outer function
===config===
<mir>
  <issueHandlers>
    <MissingClosureReturnType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function outerFn(string $x): void {
//               ^^^^^^^^^ UnusedParam: Parameter $x is never used
    $inner = function() {
//  ^^^^^^ UnusedVariable: Variable $inner is never read
        // func_get_args() is inside a closure — does not apply to outerFn
        return func_get_args();
    };
}

outerFn('hello', 'world');
//               ^^^^^^^ TooManyArguments: Too many arguments for outerFn(): expected 1, got 2
===expect===
