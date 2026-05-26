===description===
simple instanceof without template
===file===
<?php
class File {
    public ?string $filePath = null;
}

function test(string|File $file): void {
    if ($file instanceof File) {
        $path = $file->filePath;
        if ($path === null) {
            // should not error
        }
    }
}
===expect===
