===description===
method call only valid after template narrowing
===file===
<?php
class Article {
    public function getTitle(): string {
        return "Article";
    }
}

class Photo {
    public function getThumbnail(): string {
        return "thumb.jpg";
    }
}

/**
 * @template TContent as Article|Photo
 * @param TContent $content
 */
function renderContent(Article|Photo $content): void {
    if ($content instanceof Article) {
        echo $content->getTitle();
    } elseif ($content instanceof Photo) {
//            ^^^^^^^^^^^^^^^^^^^^^^^^^ RedundantCondition: Condition of type 'bool' always evaluates the same way, so one branch is unreachable
        echo $content->getThumbnail();
    }
}

/**
 * Calling an undefined method before narrowing should error
 * @template TContent as Article|Photo
 * @param TContent $content
 */
function errorCase(Article|Photo $content): void {
    $content->undefinedMethod();
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^ UndefinedMethod: Method Article::undefinedMethod() does not exist
}
===expect===
