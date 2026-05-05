===description===
basic template narrowing through instanceof
===file===
<?php
class File {
    public ?string $filePath = null;
}

/**
 * @template TValue as string|File
 * @param TValue $file
 * @return ?string
 */
function makeFileUrl(string|File $file): ?string {
    if ($file instanceof File) {
        $path = $file->filePath;    // TValue narrowed to File → $path is string|null
        if ($path === null) {
            return null;
        }
    } else {
        $path = $file;              // TValue narrowed to string → $path is string
    }
    return '//file/' . $path;
}
===expect===
