===description===
does not report namespaced function when called
===config===
<mir>
  <findUnusedCode>true</findUnusedCode>
</mir>
===file===
<?php
namespace App;

function helper(): void {}

helper();
