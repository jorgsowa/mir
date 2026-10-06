===description===
isset short-circuit with && — no undefined error in true branch
===file===
<?php
if (isset($x) && $x->method()) {}
//               ^^^^^^^^^^^^ MixedMethodCall: Method method() called on mixed type
