===description===
`self`, `static` and `parent` in a function outside a class are compile
errors; a closure is not checked, since it may be bound to a class later.
===config===
<mir>
  <issueHandlers>
    <MissingReturnType errorLevel="suppress"/>
    <MissingClosureReturnType errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function f() {
    echo self::X;
//       ^^^^ CompileError: Cannot use "self" when no class scope is active [class-keyword-outside-class]
    return new static();
//             ^^^^^^ CompileError: Cannot use "static" when no class scope is active [class-keyword-outside-class]
}

$g = function () {
    return static::class;
};
