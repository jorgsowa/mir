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
RedundantCondition@17:14: Condition is always true/false for type 'bool'
