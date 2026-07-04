===description===
Regression for fn_header_name_span: a long block comment between `)` and `{`
used to push the function name beyond a fixed 256-byte lookback window,
causing MissingReturnType to point at the body's opening brace instead of
the function name. The span must still land on the name.
===file===
<?php
function veryLongFunctionNameForRegressionTest() /* AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA */ {
    return 1;
}
===expect===
MissingReturnType@2:9-2:46: Function veryLongFunctionNameForRegressionTest() has no return type annotation
