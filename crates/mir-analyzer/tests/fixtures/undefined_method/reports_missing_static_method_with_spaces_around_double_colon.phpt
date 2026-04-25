===file===
<?php
class Math {}
function test(): void {
    Math :: missing();
}
===expect===
UndefinedMethod: Method Math::missing() does not exist
