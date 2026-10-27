===description===
PHP 8.4+ names the trait kind in the reserved-name error.
===config===
<mir>
  <phpVersion>8.5</phpVersion>
</mir>
===file===
<?php
trait object {}
//    ^^^^^^ CompileError: Cannot use "object" as a trait name as it is reserved [keyword-as-declaration-name]
