===description===
narrowed template type persists through assignment
===file===
<?php
class Document {
    public string $content;
}

class Media {
    public string $url;
}

/**
 * @template TAsset as Document|Media
 * @param TAsset $asset
 */
function processAsset(Document|Media $asset): void {
    if ($asset instanceof Document) {
        $doc = $asset;
        echo $doc->content;
    }
}

/**
 * Narrowing in one branch doesn't affect the other
 * @template TAsset as Document|Media
 * @param TAsset $asset
 */
function branchNarrowing(Document|Media $asset): void {
    if ($asset instanceof Document) {
        $doc = $asset;
        echo $doc->content;
    } else {
        $media = $asset;
        echo $media->url;
    }
}
===expect===
MissingConstructor@2:0-2:16: Class Document has uninitialized properties but no constructor
MissingConstructor@6:0-6:13: Class Media has uninitialized properties but no constructor
