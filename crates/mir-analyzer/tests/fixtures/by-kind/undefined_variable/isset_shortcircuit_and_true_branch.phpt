===description===
isset short-circuit with && — no undefined error in true branch
===file===
<?php
if (isset($x) && $x->method()) {}
===expect===
MixedMethodCall@2:17-2:29: Method method() called on mixed type
