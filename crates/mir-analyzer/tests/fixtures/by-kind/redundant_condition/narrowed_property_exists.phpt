===description===
property access only valid after template narrowing
===file===
<?php
class Document {
    public string $name;
}

class Image {
    public int $width;
}

/**
 * @template TAsset as Document|Image
 * @param TAsset $asset
 */
function getAssetInfo(Document|Image $asset): void {
    if ($asset instanceof Document) {
        echo $asset->name;
    } elseif ($asset instanceof Image) {
        echo $asset->width;
    }
}
===expect===
MissingConstructor@2:0-2:16: Class Document has uninitialized properties but no constructor
MissingConstructor@6:0-6:13: Class Image has uninitialized properties but no constructor
RedundantCondition@17:15-17:38: Condition is always true/false for type 'bool'
