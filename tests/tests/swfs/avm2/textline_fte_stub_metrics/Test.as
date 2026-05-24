package {
import flash.display.Sprite;
import flash.text.engine.*;

[SWF(width="100", height="100")]
public class Test extends Sprite {
    public function Test() {
        var fd:FontDescription = new FontDescription();
        fd.fontName = "_sans";
        var fmt:ElementFormat = new ElementFormat(fd, 14);
        var line:TextLine = new TextBlock(new TextElement("Hello", fmt))
            .createTextLine(null, 500);

        trace("textWidth usable: " + (!isNaN(line.textWidth) && line.textWidth >= 0));
        trace("textHeight usable: " + (!isNaN(line.textHeight) && line.textHeight >= 0));
    }
}
}
