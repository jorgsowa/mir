===description===
isset short-circuit with && — no undefined error in true branch
===file===
<?php
if (isset($x) && $x->method()) {}
===expect===
MixedMethodCall@2:18-2:30: Method method() called on mixed type
