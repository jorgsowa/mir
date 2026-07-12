===description===
A free function used only through first-class-callable syntax (`helper(...)`)
must not be reported unused.
===config===
suppress=
===file===
<?php
function helper(): void {}

(helper(...))();
===expect===
