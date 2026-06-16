===description===
A Closure object satisfies a callable return type — no InvalidReturnType
===file===
<?php
function getHandler(): callable {
    return function(): void {};
}
===expect===
