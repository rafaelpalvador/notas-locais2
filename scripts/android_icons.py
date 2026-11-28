import os,glob,sys
from PIL import Image,ImageDraw
R='src-tauri/gen/android/app/src/main/res'
fg=Image.open('src-tauri/icons/android-fg.png').convert('RGBA');full=Image.open('src-tauri/icons/android-full.png').convert('RGBA')
D={'mdpi':(48,108),'hdpi':(72,162),'xhdpi':(96,216),'xxhdpi':(144,324),'xxxhdpi':(192,432)}
for d,(l,f) in D.items():
    p=f'{R}/mipmap-{d}';os.makedirs(p,exist_ok=True)
    fg.resize((f,f),Image.LANCZOS).save(f'{p}/ic_launcher_foreground.png')
    sq=full.resize((l,l),Image.LANCZOS);m=Image.new('L',(l,l),0);ImageDraw.Draw(m).rounded_rectangle((0,0,l-1,l-1),radius=int(l*.22),fill=255);o=Image.new('RGBA',(l,l));o.paste(sq,(0,0),m);o.save(f'{p}/ic_launcher.png')
    m2=Image.new('L',(l,l),0);ImageDraw.Draw(m2).ellipse((0,0,l-1,l-1),fill=255);o2=Image.new('RGBA',(l,l));o2.paste(sq,(0,0),m2);o2.save(f'{p}/ic_launcher_round.png')
for x in glob.glob(f'{R}/values*/ic_launcher_background.xml'):
    s=open(x).read();import re;open(x,'w').write(re.sub(r'#[0-9A-Fa-f]{6,8}','#0B0A10',s))
os.makedirs(f'{R}/values',exist_ok=True)
x=f'{R}/values/ic_launcher_background.xml'
if not os.path.exists(x):open(x,'w').write('<?xml version="1.0" encoding="utf-8"?>\n<resources>\n  <color name="ic_launcher_background">#0B0A10</color>\n</resources>\n')
print('android icons ok')
