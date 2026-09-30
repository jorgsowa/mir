===description===
reports missing static method with spaces around double colon
===file===
<?php
class Math {}
function test(): void {
    Math :: missing();
//  ^^^^^^^^^^^^^^^^^ UndefinedMethod: Method Math::missing() does not exist
}
===expect===
