===description===
reports missing static method with spaces around double colon
===file===
<?php
class Math {}
function test(): void {
    Math :: missing();
}
===expect===
UndefinedMethod@4:5-4:22: Method Math::missing() does not exist
