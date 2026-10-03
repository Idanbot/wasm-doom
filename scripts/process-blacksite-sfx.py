#!/usr/bin/env python3
"""Rebuild the CC0 SFX from checked-in originals and exact recipes (requires ffmpeg)."""
import array,hashlib,json,math,pathlib,re,subprocess
ROOT=pathlib.Path(__file__).resolve().parents[1]
ART=ROOT/'art/audio'; OUT=ROOT/'public/game/sfx/v2'; SR=44100

def decode(path,rate=1,start=0,duration=2,lowpass=None,highpass=45):
    filters=[f'atrim=start={start}', 'asetpts=PTS-STARTPTS',f'asetrate={SR*rate}',f'aresample={SR}',f'highpass=f={highpass}']
    if lowpass: filters.append(f'lowpass=f={lowpass}')
    data=subprocess.check_output(['ffmpeg','-v','error','-i',str(path),'-ac','1','-ar',str(SR),'-af',','.join(filters),'-f','f32le','-'])
    a=array.array('f');a.frombytes(data)
    # Remove only leading silence; preserve the transient and short room tail.
    first=next((i for i,x in enumerate(a) if abs(x)>0.008),0)
    a=a[max(0,first-88):max(0,first-88)+int(SR*duration/rate)]
    return a

def render(recipe):
    layers=[]
    for l in recipe['layers']:
        a=decode(ART/'sources'/l['source'], l.get('rate',1),l.get('start',0),l.get('duration',2),l.get('lowpass'),l.get('highpass',45))
        layers.append((int(l.get('delay',0)*SR),l.get('gain',1),a))
    size=min(int(recipe.get('duration',1.8)*SR),max(offset+len(a) for offset,g,a in layers))
    mix=array.array('f',[0])*size
    for offset,g,a in layers:
        for i in range(min(len(a),size-offset)):mix[offset+i]+=a[i]*g
    peak=max(abs(x) for x in mix); assert peak>0.001,recipe['id']
    scale=10**(recipe.get('peakDb',-6)/20)/peak
    for i in range(size):
        fade=min(1,i/44,(size-1-i)/max(1,int(SR*.035)))
        mix[i]*=scale*max(0,fade)
    for ext,codec in [('ogg',['-c:a','libvorbis','-q:a','4']),('mp3',['-c:a','libmp3lame','-b:a','128k'])]:
        subprocess.run(['ffmpeg','-v','error','-y','-f','f32le','-ar',str(SR),'-ac','1','-i','-',*codec,str(OUT/(recipe['id']+'.'+ext))],input=mix.tobytes(),check=True)
    return {'duration':round(size/SR,4),'pcmPeakDb':recipe.get('peakDb',-6),'rmsDb':round(20*math.log10(math.sqrt(sum(x*x for x in mix)/size)),2),'files':{ext:hashlib.sha256((OUT/(recipe['id']+'.'+ext)).read_bytes()).hexdigest() for ext in ['ogg','mp3']}}

def main():
    m=json.loads((ART/'manifest.json').read_text()); OUT.mkdir(exist_ok=True,parents=True); true_peaks={}
    for r in m['clips']:
        r['measurements']=render(r)
        report=subprocess.run(['ffmpeg','-v','info','-i',str(OUT/(r['id']+'.ogg')),'-af','ebur128=peak=true','-f','null','-'],capture_output=True,text=True,check=True)
        true_peak=float(re.findall(r'Peak:\s*([-\d.]+) dBFS',report.stderr)[-1])
        assert true_peak <= -3, f"True peak too high: {r['id']} {true_peak}"
        true_peaks[r['id']]={'truePeakDb':true_peak,'sha256':r['measurements']['files']['ogg']}
        print(r['id'],r['measurements']['duration'],flush=True)
    (ART/'manifest.json').write_text(json.dumps(m,indent=2)+'\n')
    (ART/'true-peaks.json').write_text(json.dumps(true_peaks,indent=2)+'\n')
    (OUT/'manifest.json').write_text(json.dumps({'version':2,'clips':[{'id':r['id'],'url':'/game/sfx/v2/'+r['id']+'.ogg','fallback':'/game/sfx/v2/'+r['id']+'.mp3','vocal':r.get('vocal',False)} for r in m['clips']]},indent=2)+'\n')
if __name__=='__main__':main()
