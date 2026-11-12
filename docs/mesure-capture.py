import zlib, struct, sys
from collections import Counter
def lire_png(p):
    d=open(p,'rb').read(); i=8; idat=b''; plte=None
    while i<len(d):
        n,t=struct.unpack('>I4s',d[i:i+8]); c=d[i+8:i+8+n]; i+=12+n
        if t==b'IHDR': w,h,bd,ct=struct.unpack('>IIBB',c[:10])
        elif t==b'IDAT': idat+=c
    assert bd==8 and ct in (2,6),(bd,ct)
    bpp=3 if ct==2 else 4
    raw=zlib.decompress(idat); stride=w*bpp; rows=[]; prev=bytearray(stride); k=0
    for y in range(h):
        f=raw[k]; line=bytearray(raw[k+1:k+1+stride]); k+=1+stride
        for x in range(stride):
            a=line[x-bpp] if x>=bpp else 0; b=prev[x]; c=prev[x-bpp] if x>=bpp else 0
            if f==1: line[x]=(line[x]+a)&255
            elif f==2: line[x]=(line[x]+b)&255
            elif f==3: line[x]=(line[x]+(a+b)//2)&255
            elif f==4:
                pa=abs(b-c); pb=abs(a-c); pc=abs(a+b-2*c)
                pr=a if pa<=pb and pa<=pc else (b if pb<=pc else c)
                line[x]=(line[x]+pr)&255
        rows.append(line); prev=line
    return w,h,bpp,rows
w,h,bpp,rows=lire_png(sys.argv[1])
print('taille',w,h,'canaux',bpp)
# zone de jeu : (320*K)x(180*K) centrée ; K = 2e argument (3 par défaut)
K=int(sys.argv[2]) if len(sys.argv)>2 else 3; gw,gh=320*K,180*K; x0=(w-gw)//2; y0=(h-gh)//2
print('zone de jeu',x0,y0,gw,gh)
cnt=Counter(); semi=0
for y in range(y0,y0+gh):
    r=rows[y]
    for x in range(x0,x0+gw):
        px=tuple(r[x*bpp:x*bpp+bpp])
        cnt[px[:3]]+=1
        if bpp==4 and px[3]!=255: semi+=1
print('couleurs distinctes (zone de jeu):',len(cnt),'| pixels semi-transparents:',semi)
PAL=["10141F","1C2336","2D3A54","46587A","7F93B5","A9B8D0","C3D1E6","F3F1EA","FF5A4F","FF9A7A","FFB347","FFD84A","FFF1A8","62D0E8","2F8F9D","5B4A8A"]
pal={tuple(int(c[i:i+2],16) for i in (0,2,4)) for c in PAL}
hors=[(c,n) for c,n in cnt.items() if c not in pal]
print('couleurs hors palette:',len(hors), sorted(hors,key=lambda t:-t[1])[:5])
print('pixels hors palette:',sum(n for _,n in hors),'sur',gw*gh)
# netteté : chaque pixel logique = bloc KxK uniforme ?
bad=0; tot=0
for ly in range(180):
    for lx in range(320):
        ref=tuple(rows[y0+ly*K][ (x0+lx*K)*bpp:(x0+lx*K)*bpp+3])
        ok=all(tuple(rows[y0+ly*K+dy][(x0+lx*K+dx)*bpp:(x0+lx*K+dx)*bpp+3])==ref for dy in range(K) for dx in range(K))
        tot+=1; bad+= (not ok)
print('blocs 3x3 non uniformes (flou):',bad,'sur',tot)
# bandes
print('bande haute/bas, gauche/droite sombres :', tuple(rows[2][5*bpp:5*bpp+3]), tuple(rows[h//2][5*bpp:5*bpp+3]))
