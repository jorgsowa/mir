===description===
A trait named like a reserved type is a compile error, as for classes and interfaces.
===config===
<mir>
  <phpVersion>8.3</phpVersion>
</mir>
===file===
<?php
namespace App;

trait Mixed {}
//    ^^^^^ CompileError: Cannot use 'Mixed' as class name as it is reserved [keyword-as-declaration-name]
trait iterable {}
//    ^^^^^^^^ CompileError: Cannot use 'iterable' as class name as it is reserved [keyword-as-declaration-name]
trait Never {}
//    ^^^^^ CompileError: Cannot use 'Never' as class name as it is reserved [keyword-as-declaration-name]
trait Numeric {}
