===description===
A free function used only through first-class-callable syntax (`helper(...)`)
must not be reported unused.
===config===
<mir>
  <findUnusedCode>true</findUnusedCode>
</mir>
===file===
<?php
function helper(): void {}

(helper(...))();
===expect===
