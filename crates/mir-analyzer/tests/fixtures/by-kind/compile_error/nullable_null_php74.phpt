===description===
`null` is not a type before PHP 8.0, so `?null` names a reserved class.
===config===
<mir>
  <phpVersion>7.4</phpVersion>
</mir>
===file===
<?php
function f(?null $a): void {}
//          ^^^^ ParseError: 'null type' requires PHP 8.0 or higher [version-too-low]
//          ^^^^ CompileError: Cannot use 'null' as class name as it is reserved [invalid-nullable-type]
//         ^^^^^^^^ UnusedParam: Parameter $a is never used
