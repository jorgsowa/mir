===description===
Deprecated function attr
===file===
<?php
#[Deprecated]
function a(): void {}
a();

===expect===
DeprecatedCall@4:1-4:4: Call to deprecated function a
