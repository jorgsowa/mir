===description===
empty short-circuit with || operator — demonstrates different semantics than isset
empty($x) || $x->method() should error: empty() doesn't guarantee variable is defined
===file===
<?php
if (empty($x) || $x->method()) {
    // empty() doesn't provide same narrowing as !isset() because empty($undefined) is true
    // So this WILL error with UndefinedVariable - expected
}
===expect===
MixedMethodCall@2:17-2:29: Method method() called on mixed type
