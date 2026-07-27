===description===
A bare directive as a TRAILING comment on the same line as a `#[Attr]`
(`#[BadAttrClass] // @mir-ignore UndefinedAttributeClass`) never suppressed
the attribute's own diagnostic — `find_comment_introducer` treated the
attribute's own leading `#` as a `#`-style line-comment introducer, so
`extract_comment` returned the WHOLE line as the comment with
`has_code_before` wrongly `false`. A bare directive's scope only resolves to
`SameLine` when `has_code_before` is true, so it stayed the `NextLine`
default and targeted the declaration below instead of the attribute line
the diagnostic actually fires on.
===file===
<?php
#[BadAttrClass] // @mir-ignore UndefinedAttributeClass
class Foo {}
===expect===
