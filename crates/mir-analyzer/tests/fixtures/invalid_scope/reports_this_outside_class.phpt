===description===
reports this outside class
===file===
<?php
function test(): void {
    $this->close();
}
===expect===
InvalidScope: $this cannot be used outside of a class
===ignore===
TODO
