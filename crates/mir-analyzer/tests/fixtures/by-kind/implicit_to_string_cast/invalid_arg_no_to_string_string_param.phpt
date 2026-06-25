===description===
InvalidArgument when object has neither __toString nor Stringable and is passed to a string param
===config===
suppress=UnusedParam
===file===
<?php
class Opaque {}

function render(string $s): void {}

render(new Opaque());
===expect===
InvalidArgument@6:7-6:19: Argument $s of render() expects 'string', got 'Opaque'
