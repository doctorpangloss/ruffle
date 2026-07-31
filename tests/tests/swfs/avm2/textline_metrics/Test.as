package {
import flash.display.Sprite;
import flash.text.engine.*;

public class Test extends Sprite {
    [Embed(source="TestFont.ttf", fontName="TestFont", embedAsCFF="true", unicodeRange="U+0061-U+0064")]
    private var TestFont:Class;

    private static const SIZES:Array = [10, 20, 40];

    public function Test() {
        for each (var size:Number in SIZES) {
            var line:TextLine = lineAt(size);
            trace("size=" + size
                + " ascent=" + line.ascent
                + " descent=" + line.descent
                + " totalAscent=" + line.totalAscent
                + " totalDescent=" + line.totalDescent);
        }
    }

    private function lineAt(size:Number):TextLine {
        var fd:FontDescription = new FontDescription();
        fd.fontName = "TestFont";
        fd.fontLookup = FontLookup.EMBEDDED_CFF;

        var block:TextBlock = new TextBlock(new TextElement("a", new ElementFormat(fd, size)));
        return block.createTextLine(null, 10000);
    }
}
}
