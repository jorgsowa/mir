===description===
reports this outside class
===file===
<?php
function test(): void {
    $this->close();
//  ^^^^^ InvalidScope: $this cannot be used outside of a class
}
===expect===
