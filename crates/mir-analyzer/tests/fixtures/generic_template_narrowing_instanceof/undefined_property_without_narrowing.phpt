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
