===description===
reports this outside class
===file===
<?php
function test(): void {
    $this->close();
}
===expect===
InvalidScope@3:4: $this cannot be used outside of a class
