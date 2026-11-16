---
title: UnrestrictedFileUpload
code: MIR0826
description: "An uploaded file is saved under the name the client sent, with no check of its extension or content."
sidebar:
  hidden: true
  order: 826
---

`move_uploaded_file()` saves the upload at a path that ends in the client's file name, `$_FILES[...]['name']`, so the client picks the extension: a `shell.php` uploaded into the web root runs as code (CWE-434). `basename()` stops path traversal but keeps the extension. Reported when the destination ends in the client name, directly, through `basename()`, `strtolower()`, `str_replace()` and similar, or through a variable holding it, and nothing earlier on the path checks the extension or content: `pathinfo()`, `getimagesize()`, `exif_imagetype()`, `finfo_file()`, `mime_content_type()`, `wp_check_filetype()`, a string function or pattern (`substr()`, `explode()`, `preg_match()`, `str_ends_with()`, `in_array()`) applied to the client's name, or a helper of the program's own given the upload in a condition. A check of `$_FILES[...]['type']` alone doesn't count, since the client sends that value too.

## Example

```php
<?php
$target = 'uploads/' . basename($_FILES['file']['name']);
move_uploaded_file($_FILES['file']['tmp_name'], $target); // UnrestrictedFileUpload
```

## How to fix

Check the extension against an allowlist, or better, generate the file name yourself and pick the extension from the detected content type: `'uploads/' . bin2hex(random_bytes(16)) . '.jpg'`.
