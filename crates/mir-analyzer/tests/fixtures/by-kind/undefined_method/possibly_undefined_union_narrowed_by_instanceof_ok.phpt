===description===
Narrowing the union to the declaring member leaves no diagnostics; a
non-union receiver lacking the method stays an Error.
===file===
<?php
class Reader {
    public function read(): void {}
}
class Writer {}
function narrowed(Reader|Writer $io): void {
    if ($io instanceof Reader) {
        $io->read();
    }
}
function plain(Writer $w): void {
    $w->read();
//  ^^^^^^^^^^ UndefinedMethod: Method Writer::read() does not exist
}
