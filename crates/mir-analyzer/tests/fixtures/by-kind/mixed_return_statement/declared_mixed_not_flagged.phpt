===description===
When declared return type is mixed, MixedReturnStatement does NOT fire even when the returned value is mixed
===file===
<?php
function decode(): mixed {
    return json_decode('{"key":"value"}');
}
===expect===
