===description===
`mixed` is reserved from PHP 8.0 and `never` from 8.1; before, they are ordinary trait names.
===config===
<mir>
  <phpVersion>7.4</phpVersion>
</mir>
===file===
<?php
trait Mixed {}
trait Never {}
trait Void {}
//    ^^^^ CompileError: Cannot use 'Void' as class name as it is reserved [keyword-as-declaration-name]
