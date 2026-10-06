===description===
undefined property error when template not narrowed correctly
===file===
<?php
class File {
//<^^^^^^^^^^^^ MissingConstructor: Class File has uninitialized properties but no constructor
    public string $path;
}

class Stream {
//<^^^^^^^^^^^^^^ MissingConstructor: Class Stream has uninitialized properties but no constructor
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
