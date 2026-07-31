package {
import flash.display.Sprite;
import flash.text.engine.*;

public class Test extends Sprite {
    private static const CHAR_INDICES:Array = [-2, -1, 0, 1, 2, 3, 4, 5, 100];

    public function Test() {
        var block:TextBlock = new TextBlock(new TextElement("ab\ncd", getElementFormat()));
        var line:TextLine = block.createTextLine(null, 10000);
        var index:int = 0;
        while (line) {
            trace("line " + index
                + " textBlockBeginIndex=" + line.textBlockBeginIndex
                + " rawTextLength=" + line.rawTextLength
                + " atomCount=" + line.atomCount);
            for each (var charIndex:int in CHAR_INDICES) {
                trace("  getAtomIndexAtCharIndex(" + charIndex + ")="
                    + line.getAtomIndexAtCharIndex(charIndex));
            }
            line = block.createTextLine(line, 10000);
            index++;
        }
    }

    private function getElementFormat():ElementFormat {
        return new ElementFormat();
    }
}
}
