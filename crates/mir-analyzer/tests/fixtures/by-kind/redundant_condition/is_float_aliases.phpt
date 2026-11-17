===description===
is_float, is_double, is_real all narrow to float type
===config===
<mir>
  <issueHandlers>
    <MissingParamType errorLevel="suppress"/>
    <MissingReturnType errorLevel="suppress"/>
    <MixedArgument errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function testIsFloat($x) {
    if (is_float($x)) {
        return $x + 1.5;
    }
}

function testIsDouble($x) {
    if (is_double($x)) {
//      ^^^^^^^^^^^^^ DeprecatedCall: Call to deprecated function is_double
        return $x * 2.0;
    }
}

function testIsReal($x) {
    if (is_real($x)) {
//      ^^^^^^^^^^^ DeprecatedCall: Call to deprecated function is_real
        return sqrt($x);
    }
}
