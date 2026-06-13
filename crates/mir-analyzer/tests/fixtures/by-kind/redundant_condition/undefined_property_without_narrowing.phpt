===description===
undefined property error when template not narrowed correctly
===file===
<?php
class File {
    public string $path;
}

class Stream {
    public int $handle;
}

/**
 * @template TResource as File|Stream
 * @param TResource $resource
 */
function processResource(File|Stream $resource): void {
    if ($resource instanceof File) {
        echo $resource->path;
    } else {
        echo $resource->handle;
    }

}
===expect===
MissingConstructor@2:0-2:12: Class File has uninitialized properties but no constructor
MissingConstructor@6:0-6:14: Class Stream has uninitialized properties but no constructor
