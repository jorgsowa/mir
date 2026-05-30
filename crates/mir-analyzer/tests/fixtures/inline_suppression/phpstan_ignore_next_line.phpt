===description===
@phpstan-ignore-next-line suppresses all issues on the next line
===file===
<?php
function test(): void {
    // @phpstan-ignore-next-line
    new NoSuchClass();
}
===expect===
