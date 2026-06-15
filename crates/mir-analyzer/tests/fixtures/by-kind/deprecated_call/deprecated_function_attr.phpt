===description===
Deprecated function attr
===file===
<?php
#[Deprecated]
function a(): void {}
a();

===expect===
DeprecatedCall@4:0-4:3: Call to deprecated function a
