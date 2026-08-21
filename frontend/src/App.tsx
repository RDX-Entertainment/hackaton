import { FormEvent, useEffect, useState } from 'react';
import { Link, Navigate, Route, Routes, useLocation, useNavigate, useParams } from 'react-router-dom';
import { useDispatch, useSelector } from 'react-redux';
import { api, bitcoinApi, clanApi, financeApi, gameApi, hardwareApi, internetApi, listApi, mailApi, missionsApi, researchApi, softwareApi, type Dashboard, type Machine, type OwnedFile, type Remote, type RemoteFile, type ShopPart, type Target, type Visited } from './api';
import { authenticated, signedOut, type RootState } from './store';

const pages = [
  ['software','Software'], ['internet','Internet'], ['processes','Processes'], ['missions','Missions'], ['hardware','Hardware'], ['clan','Clan'], ['list','List'], ['finances','Finances'], ['university','University'], ['ranking','Ranking'], ['log','Log'], ['fame','Fame'], ['doom','Doom'], ['bitcoin','Bitcoin'], ['certs','Certifications'], ['mail','Mail'], ['profile','Profile'], ['settings','Settings'], ['research','Research'], ['riddle','Riddle'], ['war','War'], ['webserver','Webserver'], ['stats','Stats'],
] as const;

export function App() { const token = useSelector((s: RootState) => s.auth.token); return <Routes><Route path="/login" element={<Login/>}/><Route path="/register" element={<Register/>}/><Route path="/*" element={token ? <Game/> : <Navigate to="/login" replace/>}/></Routes>; }
function AuthForm({register = false}:{register?:boolean}) { const dispatch = useDispatch(); const nav = useNavigate(); const [error,setError] = useState(''); const [username,setUsername] = useState(''); const [password,setPassword] = useState(''); const [email,setEmail] = useState(''); const submit = async (event: FormEvent) => { event.preventDefault(); try { const result = register ? await api.register(username,password,email) : await api.login(username,password); dispatch(authenticated(result)); nav('/'); } catch (e) { setError(e instanceof Error ? e.message : 'Request failed'); } }; return <main className="auth"><h1>Hacker Experience</h1><p>The Internet under attack<span>_</span></p><form onSubmit={submit}><label>Username<input value={username} onChange={e=>setUsername(e.target.value)} required /></label>{register && <label>Email<input type="email" value={email} onChange={e=>setEmail(e.target.value)} required /></label>}<label>Password<input type="password" value={password} onChange={e=>setPassword(e.target.value)} required /></label>{error && <output>{error}</output>}<button>{register?'Register':'Login'}</button></form><Link to={register?'/login':'/register'}>{register?'Already registered? Login':'Create an account'}</Link></main> }
function Login(){return <AuthForm/>} function Register(){return <AuthForm register/>}
function Game(){ const dispatch = useDispatch(); const user = useSelector((s:RootState)=>s.auth.user); return <div className="shell"><header><Link to="/">Hacker Experience</Link><span>{user?.login}</span><button onClick={()=>dispatch(signedOut())}>Logout</button></header><aside>{pages.map(([path,label])=><Link key={path} to={`/${path}`}>{label}</Link>)}<Link to="/wiki">Wiki</Link><Link to="/forum">Forum</Link></aside><main className="game"><Routes><Route index element={<Home/>}/><Route path="wiki" element={<Wiki/>}/><Route path="wiki/:slug" element={<WikiPage/>}/><Route path="forum" element={<Forum/>}/><Route path="forum/:id" element={<ForumThread/>}/>{pages.map(([path,label])=><Route key={path} path={path} element={<LegacyPage page={path} label={label}/>}/>)}</Routes></main></div> }
function Home(){const [data,setData]=useState<Dashboard>();const [news,setNews]=useState<{id:number;title:string;author:string;content:string}[]>([]); const [error,setError]=useState(''); useEffect(()=>{api.dashboard().then(setData).catch(e=>setError(e.message));api.news().then(setNews)},[]); if(error)return <p className="error">{error}</p>; if(!data)return <p>Loading...</p>; return <><h2>Home</h2><section className="cards"><article><b>${data.money.toFixed(2)}</b><small>Total money</small></article><article><b>{data.processes}</b><small>Processes</small></article><article><b>{data.unread_mail}</b><small>Unread mail</small></article></section><h3>News</h3>{news.length?news.map(n=><article className="forum-post" key={n.id}><b>{n.title}</b><small> by {n.author}</small><pre>{n.content}</pre></article>):<p className="muted">No news yet.</p>}</>}
const resources: Record<string,string> = { software:'/software', hardware:'/hardware', internet:'/internet/targets', processes:'/processes', mail:'/mails', missions:'/missions', finances:'/finances/accounts', ranking:'/ranking/users', clan:'/clans', research:'/research', bitcoin:'/bitcoin/wallets', list:'/lists', log:'/logs' };
function LegacyPage({page,label}:{page:string;label:string}) { if(page==='internet')return <InternetScreen/>;if(page==='software')return <SoftwareScreen/>;if(page==='hardware')return <HardwareScreen/>;if(page==='processes')return <ProcessesScreen/>;if(page==='missions')return <MissionsScreen/>;if(page==='mail')return <MailScreen/>;if(page==='university'||page==='certs')return <UniversityScreen/>;if(page==='research')return <ResearchScreen/>;if(page==='profile')return <ProfileScreen/>;if(page==='settings')return <SettingsScreen/>;if(page==='stats')return <StatsScreen/>;if(page==='finances')return <FinancesScreen/>;if(page==='clan')return <ClanScreen/>;if(page==='bitcoin')return <BitcoinScreen/>;if(page==='list')return <ListScreen/>;const location=useLocation(); const [rows,setRows]=useState<Record<string,unknown>[] | null>(null); const [status,setStatus]=useState('Loading...'); const reload=()=>{ const load=resources[page] ? api.resource(resources[page]) : api.page(page); load.then((d)=>{ if(Array.isArray(d)) setRows(d); else setStatus(d.status); }).catch(e=>setStatus(e.message))}; useEffect(()=>{void reload()},[page]); return <><h2>{label}</h2>{page==='mail' && <Compose onSent={reload}/>} {page==='finances' && <Transfer onSent={reload}/>} {rows ? <DataTable rows={rows}/> : <p className="muted">{status} · {location.search || 'default view'}</p>}</> }
function MailScreen(){const [rows,setRows]=useState<Record<string,unknown>[]>([]);const [selected,setSelected]=useState<Record<string,unknown>>();const [message,setMessage]=useState('');const load=()=>mailApi.list().then(setRows);useEffect(()=>{void load()},[]);const read=async(id:number)=>{try{setSelected(await mailApi.read(id));load()}catch(err){setMessage(err instanceof Error?err.message:'Request failed')}};const remove=async(id:number)=>{try{await mailApi.remove(id);setSelected(undefined);setMessage('Mail deleted.');load()}catch(err){setMessage(err instanceof Error?err.message:'Request failed')}};return <><h2>Mail</h2><Compose onSent={load}/><p className="muted">{message}</p>{selected&&<article className="forum-post"><b>{String(selected.subject)}</b><pre>{String(selected.text)}</pre><button onClick={()=>void remove(Number(selected.id))}>Delete</button></article>}{rows.map(m=><article className="forum-post" key={String(m.id)}><button onClick={()=>void read(Number(m.id))}>{String(m.subject)}</button><small> from {String(m.from)} · {String(m.read?'read':'unread')}</small></article>)}</>}
function MissionsScreen(){const [rows,setRows]=useState<Record<string,unknown>[]>([]);const [message,setMessage]=useState('');const load=()=>missionsApi.list().then(setRows);useEffect(()=>{void load()},[]);const command=async(id:number,action:'accept'|'abort')=>{try{await missionsApi.command(id,action);setMessage(`Mission ${action}ed.`);load()}catch(err){setMessage(err instanceof Error?err.message:'Request failed')}};return <><h2>Missions</h2><p className="muted">{message}</p>{rows.map(m=><article className="forum-post" key={String(m.id)}><b>Mission #{String(m.id)} · level {String(m.level)}</b><p>Prize: ${String(m.prize)} · status {String(m.status)}</p>{Number(m.status)===1?<button onClick={()=>void command(Number(m.id),'accept')}>Accept</button>:Number(m.status)===2?<button onClick={()=>void command(Number(m.id),'abort')}>Abort</button>:null}</article>)}</>}
function ResearchScreen(){const [rows,setRows]=useState<Record<string,unknown>[]>([]);const [software,setSoftware]=useState('');const [message,setMessage]=useState('');const load=()=>api.resource('/research').then(setRows);useEffect(()=>{void load()},[]);const start=async(e:FormEvent)=>{e.preventDefault();try{const r=await researchApi.start(Number(software));setSoftware('');setMessage(`Research queued as task #${r.process_id}. Run the worker to apply it.`);load()}catch(err){setMessage(err instanceof Error?err.message:'Request failed')}};return <><h2>Research</h2><form className="compose" onSubmit={start}><label>Owned software ID<input value={software} inputMode="numeric" onChange={e=>setSoftware(e.target.value)} required/></label><button>Research next version</button><span className="muted">{message}</span></form><DataTable rows={rows}/></>}
function FinancesScreen(){const [accounts,setAccounts]=useState<Record<string,unknown>[]>([]);const [bank,setBank]=useState('1');const [pin,setPin]=useState('');const [message,setMessage]=useState('');const load=()=>api.resource('/finances/accounts').then(setAccounts);useEffect(()=>{void load()},[]);const open=async(e:FormEvent)=>{e.preventDefault();try{const r=await financeApi.openAccount(Number(bank),pin);setPin('');setMessage(`Account ${r.account} opened.`);load()}catch(err){setMessage(err instanceof Error?err.message:'Request failed')}};return <><h2>Finances</h2><form className="compose" onSubmit={open}><label>Bank ID<input value={bank} inputMode="numeric" onChange={e=>setBank(e.target.value)} required/></label><label>New 6-character PIN<input value={pin} minLength={6} maxLength={6} onChange={e=>setPin(e.target.value)} required/></label><button>Open account</button><span className="muted">{message}</span></form><Transfer onSent={load}/><DataTable rows={accounts}/></>}
function StatsScreen(){const [stats,setStats]=useState<Record<string,unknown>>();useEffect(()=>{api.serverStats().then(setStats)},[]);if(!stats)return <p>Waiting for the game worker…</p>;return <><h2>Server statistics</h2><DataTable rows={[stats]}/></>}
function SettingsScreen(){const [data,setData]=useState<{login:string;email:string;premium:boolean}>();const [email,setEmail]=useState('');const [current,setCurrent]=useState('');const [next,setNext]=useState('');const [message,setMessage]=useState('');const load=()=>api.settings().then(s=>{setData(s);setEmail(s.email)});useEffect(()=>{void load()},[]);const save=async(e:FormEvent)=>{e.preventDefault();try{await api.saveSettings(email,current,next);setCurrent('');setNext('');setMessage('Settings saved.');load()}catch(err){setMessage(err instanceof Error?err.message:'Request failed')}};if(!data)return <p>Loading…</p>;return <><h2>Settings · {data.login}</h2><form className="compose" onSubmit={save}><label>Email<input type="email" value={email} onChange={e=>setEmail(e.target.value)} required/></label><label>Current password<input type="password" value={current} onChange={e=>setCurrent(e.target.value)} required/></label><label>New password (optional)<input type="password" value={next} minLength={8} onChange={e=>setNext(e.target.value)}/></label><button>Save settings</button><span className="muted">{message}</span></form></>}
function UniversityScreen(){const [data,setData]=useState<{items:{level:number;name:string;description:string;price:number;completed:boolean;available:boolean;learning:boolean}[]}>();const [accounts,setAccounts]=useState<Record<string,unknown>[]>([]);const [account,setAccount]=useState('');const [message,setMessage]=useState('');const load=()=>api.certifications().then(setData);useEffect(()=>{void load();api.resource('/finances/accounts').then(setAccounts)},[]);const act=async(level:number,learning:boolean,price:number)=>{try{if(learning)await api.completeCertification(level);else await api.startCertification(level,price?Number(account):undefined);setMessage('Certification updated.');load()}catch(err){setMessage(err instanceof Error?err.message:'Request failed')}};if(!data)return <p>Loading…</p>;return <><h2>University</h2><p className="muted">{message}</p>{data.items.some(c=>c.price>0&&!c.completed)&&<label>Payment account<select value={account} onChange={e=>setAccount(e.target.value)}><option value="">Select an account</option>{accounts.map(a=><option key={String(a.account)} value={String(a.account)}>{String(a.account)} · ${String(a.money)}</option>)}</select></label>}{data.items.map(c=><article className="forum-post" key={c.level}><b>{c.level}. {c.name}</b><p>{c.description} · ${c.price}</p>{c.completed?<small>Completed</small>:c.learning?<button onClick={()=>void act(c.level,true,c.price)}>Complete</button>:c.available?<button disabled={c.price>0&&!account} onClick={()=>void act(c.level,false,c.price)}>Start</button>:<small>Locked</small>}</article>)}</>}
function ProcessesScreen(){const [rows,setRows]=useState<Record<string,unknown>[]>([]);const [actions,setActions]=useState<{id:number;name:string}[]>([]);const [action,setAction]=useState('1');const [victim,setVictim]=useState('');const [info,setInfo]=useState('');const [message,setMessage]=useState('');const load=()=>api.resource('/processes').then(setRows);useEffect(()=>{void load();gameApi.processActions().then(setActions)},[]);const start=async(e:FormEvent)=>{e.preventDefault();try{await api.startProcess(Number(action),victim?Number(victim):undefined,undefined,info);setInfo('');setMessage('Task started. Run the worker to advance queued tasks.');load()}catch(err){setMessage(err instanceof Error?err.message:'Request failed')}};const command=async(id:number,action:'pause'|'resume'|'delete')=>{try{await gameApi.processCommand(id,action);setMessage(`Task ${action}d.`);load()}catch(err){setMessage(err instanceof Error?err.message:'Request failed')}};return <><h2>Processes</h2><form className="compose" onSubmit={start}><label>Action<select value={action} onChange={e=>setAction(e.target.value)}>{actions.map(a=><option key={a.id} value={a.id}>{a.name} ({a.id})</option>)}</select></label><label>Victim ID (optional)<input value={victim} inputMode="numeric" onChange={e=>setVictim(e.target.value)}/></label><label>Info<input value={info} onChange={e=>setInfo(e.target.value)} maxLength={30}/></label><button>Start task</button><span className="muted">{message}</span></form>{rows.map(p=><article className="forum-post" key={String(p.id)}><b>Task #{String(p.id)} · {String(p.details||p.action)}</b><p>{String(p.worked_seconds)}/{String(p.ideal_seconds)} seconds · {p.completed?'completed':p.paused?'paused':'running'}</p>{!p.completed&&<>{p.paused?<button onClick={()=>void command(Number(p.id),'resume')}>Resume</button>:<button onClick={()=>void command(Number(p.id),'pause')}>Pause</button>}<button onClick={()=>void command(Number(p.id),'delete')}>Delete</button></>}</article>)}</>}
function ProfileScreen(){const [data,setData]=useState<{login:string;game_ip:number;biography:string;visible:boolean;reputation:number;rank:number;missions:number;hacks:number;money_earned:number}>();const [bio,setBio]=useState('');const [visible,setVisible]=useState(true);const [message,setMessage]=useState('');const load=()=>api.profile().then(p=>{setData(p);setBio(p.biography);setVisible(p.visible)});useEffect(()=>{void load()},[]);const save=async(e:FormEvent)=>{e.preventDefault();try{await api.saveProfile(bio,visible);setMessage('Profile saved.');load()}catch(err){setMessage(err instanceof Error?err.message:'Request failed')}};if(!data)return <p>Loading…</p>;return <><h2>Profile · {data.login}</h2><section className="cards"><article><b>{data.game_ip}</b><small>Game IP</small></article><article><b>{data.rank}</b><small>Rank</small></article><article><b>{data.reputation}</b><small>Reputation</small></article><article><b>{data.missions}</b><small>Missions</small></article></section><form className="compose" onSubmit={save}><label>Biography<textarea value={bio} onChange={e=>setBio(e.target.value)} maxLength={3000}/></label><label><input type="checkbox" checked={visible} onChange={e=>setVisible(e.target.checked)}/> Public profile</label><button>Save profile</button><span className="muted">{message}</span></form></>}
function ClanScreen(){const [clan,setClan]=useState<{name:string;nick:string;description:string;auth_level:number;members:{id:number;login:string;auth_level:number}[]}|null|undefined>();const [all,setAll]=useState<{id:number;name:string;nick:string;members:number;slots:number;description:string}[]>([]);const [pending,setPending]=useState<{id:number;login:string;message:string;created_at:string}[]>([]);const [name,setName]=useState('');const [nick,setNick]=useState('');const [description,setDescription]=useState('');const [request,setRequest]=useState('');const [message,setMessage]=useState('');const load=()=>api.myClan().then(c=>{setClan(c);if(c?.auth_level&&c.auth_level>=2)clanApi.requests().then(setPending)});useEffect(()=>{void load();clanApi.all().then(setAll)},[]);const create=async(e:FormEvent)=>{e.preventDefault();try{await api.createClan(name,nick,description);setMessage('Clan created.');load()}catch(err){setMessage(err instanceof Error?err.message:'Request failed')}};const join=async(id:number)=>{try{await clanApi.join(id,request);setMessage('Join request sent.')}catch(err){setMessage(err instanceof Error?err.message:'Request failed')}};const accept=async(id:number)=>{try{await clanApi.accept(id);setMessage('Request accepted.');load()}catch(err){setMessage(err instanceof Error?err.message:'Request failed')}};if(clan===undefined)return <p>Loading…</p>;if(clan)return <><h2>[{clan.nick}] {clan.name}</h2><p>{clan.description}</p><p className="muted">{message}</p><DataTable rows={clan.members}/>{clan.auth_level>=2&&<><h3>Join requests</h3>{pending.length?pending.map(p=><article className="forum-post" key={p.id}><b>{p.login}</b><p>{p.message}</p><button onClick={()=>void accept(p.id)}>Accept</button></article>):<p className="muted">No pending requests.</p>}</>}</>;return <><h2>Clan</h2><form className="compose" onSubmit={create}><label>Name<input value={name} maxLength={25} onChange={e=>setName(e.target.value)} required/></label><label>Tag (3 characters)<input value={nick} maxLength={3} onChange={e=>setNick(e.target.value)} required/></label><label>Description<textarea value={description} onChange={e=>setDescription(e.target.value)}/></label><button>Create clan</button></form><label>Message for a join request<input value={request} onChange={e=>setRequest(e.target.value)}/></label><span className="muted">{message}</span>{all.map(c=><article className="forum-post" key={c.id}><b>[{c.nick}] {c.name}</b><p>{c.description} · {c.members}/{c.slots}</p><button onClick={()=>void join(c.id)}>Request join</button></article>)}</>}
function BitcoinScreen(){const [wallets,setWallets]=useState<{address:string;amount:string}[]>([]);const [from,setFrom]=useState('');const [to,setTo]=useState('');const [amount,setAmount]=useState('');const [message,setMessage]=useState('');const load=()=>api.wallets().then(setWallets);useEffect(()=>{void load()},[]);const create=async()=>{try{await api.createWallet();setMessage('Wallet created.');load()}catch(err){setMessage(err instanceof Error?err.message:'Request failed')}};const transfer=async(e:FormEvent)=>{e.preventDefault();try{await bitcoinApi.transfer(from,to,amount);setAmount('');setMessage('Bitcoin transfer complete.');load()}catch(err){setMessage(err instanceof Error?err.message:'Request failed')}};return <><h2>Bitcoin</h2><button onClick={()=>void create()}>Create wallet</button><form className="compose" onSubmit={transfer}><label>From wallet<select value={from} onChange={e=>setFrom(e.target.value)} required><option value="">Select wallet</option>{wallets.map(w=><option key={w.address} value={w.address}>{w.address}</option>)}</select></label><label>To wallet<input value={to} onChange={e=>setTo(e.target.value)} required/></label><label>Amount<input value={amount} inputMode="decimal" onChange={e=>setAmount(e.target.value)} required/></label><button>Transfer</button></form><span className="muted"> {message}</span><DataTable rows={wallets}/></>}
function ListScreen(){const [entries,setEntries]=useState<{id:number;ip:number;login:string;hacked_at:string;virus_id:number}[]>([]);const [ip,setIp]=useState('');const [login,setLogin]=useState('');const [password,setPassword]=useState('');const [message,setMessage]=useState('');const load=()=>api.lists().then(setEntries);useEffect(()=>{void load()},[]);const add=async(e:FormEvent)=>{e.preventDefault();try{await api.addList(Number(ip),login,password);setIp('');setLogin('');setPassword('');setMessage('Account added.');load()}catch(err){setMessage(err instanceof Error?err.message:'Request failed')}};const remove=async(id:number)=>{try{await listApi.remove(id);setMessage('Account removed.');load()}catch(err){setMessage(err instanceof Error?err.message:'Request failed')}};return <><h2>Compromised list</h2><form className="compose" onSubmit={add}><label>IP<input value={ip} inputMode="numeric" onChange={e=>setIp(e.target.value)} required/></label><label>Login<input value={login} onChange={e=>setLogin(e.target.value)} required/></label><label>Password<input value={password} onChange={e=>setPassword(e.target.value)} required/></label><button>Add account</button><span className="muted">{message}</span></form>{entries.map(entry=><article className="forum-post" key={entry.id}><b>{entry.ip} · {entry.login}</b><p>Virus #{entry.virus_id} · {entry.hacked_at}</p><button onClick={()=>void remove(entry.id)}>Delete</button></article>)}</>}
function Compose({onSent}:{onSent:()=>void}) { const [to,setTo]=useState(''); const [subject,setSubject]=useState(''); const [text,setText]=useState(''); const [message,setMessage]=useState(''); const send=async(event:FormEvent)=>{event.preventDefault();try{await api.sendMail(to,subject,text);setTo('');setSubject('');setText('');setMessage('E-mail sent.');onSent()}catch(error){setMessage(error instanceof Error?error.message:'Request failed')}};return <form className="compose" onSubmit={send}><label>To<input value={to} onChange={e=>setTo(e.target.value)} required/></label><label>Subject<input value={subject} maxLength={40} onChange={e=>setSubject(e.target.value)} required/></label><label>Message<textarea value={text} onChange={e=>setText(e.target.value)} required/></label><button>Send</button><span className="muted">{message}</span></form>}
function Transfer({onSent}:{onSent:()=>void}) { const [from,setFrom]=useState(''); const [password,setPassword]=useState(''); const [to,setTo]=useState(''); const [amount,setAmount]=useState(''); const [message,setMessage]=useState(''); const send=async(event:FormEvent)=>{event.preventDefault();try{await api.transfer(Number(from),password,Number(to),Number(amount));setPassword('');setAmount('');setMessage('Transfer complete.');onSent()}catch(error){setMessage(error instanceof Error?error.message:'Request failed')}};return <form className="compose" onSubmit={send}><label>From account<input inputMode="numeric" value={from} onChange={e=>setFrom(e.target.value)} required/></label><label>Account password<input type="password" value={password} onChange={e=>setPassword(e.target.value)} required/></label><label>Destination account<input inputMode="numeric" value={to} onChange={e=>setTo(e.target.value)} required/></label><label>Amount<input inputMode="numeric" min="1" value={amount} onChange={e=>setAmount(e.target.value)} required/></label><button>Transfer</button><span className="muted">{message}</span></form>}
function Wiki(){const [pages,setPages]=useState<{slug:string;title:string}[]>([]);useEffect(()=>{api.wikiIndex().then(setPages)},[]);return <><h2>Wiki</h2><p className="muted">Documentación colaborativa del juego.</p><p><Link to="/wiki/start">Create or edit the start page</Link></p><DataTable rows={pages}/></>}
function WikiPage(){const {slug='start'}=useParams();const [page,setPage]=useState<{title:string;content:string;author:string|null}>();const [title,setTitle]=useState('');const [content,setContent]=useState('');const [message,setMessage]=useState('');const load=()=>api.wiki(slug).then(p=>{setPage(p);setTitle(p.title);setContent(p.content)}).catch(()=>{setPage(undefined);setTitle(slug);setContent('')});useEffect(()=>{void load()},[slug]);const save=async(e:FormEvent)=>{e.preventDefault();try{await api.saveWiki(slug,title,content);setMessage('Saved.');load()}catch(err){setMessage(err instanceof Error?err.message:'Request failed')}};return <><p><Link to="/wiki">← Wiki index</Link></p><h2>{page?.title||`New page: ${slug}`}</h2>{page&&<article className="wiki-content"><pre>{page.content}</pre><small>Last author: {page.author||'unknown'}</small></article>}<h3>Edit</h3><form className="compose" onSubmit={save}><label>Title<input value={title} onChange={e=>setTitle(e.target.value)} required/></label><label>Content<textarea value={content} onChange={e=>setContent(e.target.value)} required/></label><button>Save revision</button><span className="muted">{message}</span></form></>}
function Forum(){const [topics,setTopics]=useState<{id:number;title:string;author:string;posts:number}[]>([]);const [title,setTitle]=useState('');const [content,setContent]=useState('');const [message,setMessage]=useState('');const load=()=>api.forumTopics().then(setTopics);useEffect(()=>{void load()},[]);const post=async(e:FormEvent)=>{e.preventDefault();try{await api.createTopic(title,content);setTitle('');setContent('');setMessage('Topic created.');load()}catch(err){setMessage(err instanceof Error?err.message:'Request failed')}};return <><h2>Forum</h2><form className="compose" onSubmit={post}><label>Topic title<input value={title} onChange={e=>setTitle(e.target.value)} required/></label><label>First post<textarea value={content} onChange={e=>setContent(e.target.value)} required/></label><button>Post topic</button><span className="muted">{message}</span></form><div className="table-wrap"><table><thead><tr><th>Topic</th><th>Author</th><th>Posts</th></tr></thead><tbody>{topics.map(t=><tr key={t.id}><td><Link to={`/forum/${t.id}`}>{t.title}</Link></td><td>{t.author}</td><td>{t.posts}</td></tr>)}</tbody></table></div></>}
function ForumThread(){const {id=''}=useParams();const [topic,setTopic]=useState<{title:string;author:string;posts:{id:number;author:string;content:string}[]}>();const [content,setContent]=useState('');const [message,setMessage]=useState('');const load=()=>api.forumTopic(Number(id)).then(setTopic).catch(()=>setTopic(undefined));useEffect(()=>{void load()},[id]);const reply=async(e:FormEvent)=>{e.preventDefault();try{await api.replyTopic(Number(id),content);setContent('');setMessage('Reply posted.');load()}catch(err){setMessage(err instanceof Error?err.message:'Request failed')}};if(!topic)return <p>Loading…</p>;return <><p><Link to="/forum">← Forum</Link></p><h2>{topic.title}</h2>{topic.posts.map(p=><article className="forum-post" key={p.id}><b>{p.author}</b><pre>{p.content}</pre></article>)}<form className="compose" onSubmit={reply}><label>Reply<textarea value={content} onChange={e=>setContent(e.target.value)} required/></label><button>Reply</button><span className="muted">{message}</span></form></>}
// The Internet screen. The legacy drove all of this from one page whose contents changed
// with $_SESSION: an address bar, then a login form, then the machine's insides once you
// were in. Same shape here, driven by what GET /internet/{ip} reports instead.
function InternetScreen() {
  const [address, setAddress] = useState('');
  const [remote, setRemote] = useState<Remote>();
  const [files, setFiles] = useState<RemoteFile[]>([]);
  const [log, setLog] = useState('');
  const [draft, setDraft] = useState('');
  const [targets, setTargets] = useState<Target[]>([]);
  const [history, setHistory] = useState<Visited[]>([]);
  const [user, setUser] = useState('root');
  const [password, setPassword] = useState('');
  const [message, setMessage] = useState('');
  const say = (err: unknown) => setMessage(err instanceof Error ? err.message : 'Request failed');
  const sidebars = () => { internetApi.targets().then(setTargets).catch(()=>{}); internetApi.history().then(setHistory).catch(()=>{}) };
  useEffect(() => { sidebars() }, []);
  const visit = async (ip: number) => {
    setAddress(String(ip)); setFiles([]); setLog(''); setMessage('');
    try {
      const r = await internetApi.navigate(ip);
      setRemote(r);
      if (r.access) await enter(ip);
    } catch (err) { setRemote(undefined); say(err) }
    sidebars();
  };
  // Everything past the login is only readable once access exists, so it loads together.
  const enter = async (ip: number) => {
    try { setFiles(await internetApi.files(ip)) } catch { setFiles([]) }
    try { const l = await internetApi.log(ip); setLog(l.text); setDraft(l.text) } catch { setLog('') }
  };
  const signIn = async (event: FormEvent) => {
    event.preventDefault();
    if (!remote) return;
    try { await internetApi.login(remote.ip, user, password); setPassword(''); setMessage('Access granted.'); await visit(remote.ip) } catch (err) { say(err) }
  };
  const crack = async (kind: 'bf' | 'scan') => {
    if (!remote) return;
    try { const r = await internetApi.hack(remote.ip, kind); setMessage(`Task #${r.process_id} started · yours v${r.attacker_version} against v${r.defender_version}. Run the worker, then reconnect.`) } catch (err) { say(err) }
  };
  const act = async (run: Promise<{ process_id?: number; id?: number }>, label: string) => {
    try { const r = await run; setMessage(`${label} queued as task #${r.process_id ?? r.id}. It lands when the worker runs.`) } catch (err) { say(err) }
  };
  const bookmark = async () => {
    if (!remote) return;
    try { remote.bookmarked ? await internetApi.removeBookmark(remote.ip) : await internetApi.addBookmark(remote.ip); await visit(remote.ip) } catch (err) { say(err) }
  };
  return <>
    <h2>Internet</h2>
    <form className="compose" onSubmit={e => { e.preventDefault(); void visit(Number(address)) }}>
      <label>Address<input value={address} inputMode="numeric" placeholder="Game IP" onChange={e => setAddress(e.target.value)} required /></label>
      <button>Connect</button>
      <span className="muted">{message}</span>
    </form>
    {remote && <article className="forum-post">
      <b>{remote.name || 'Unknown machine'}</b>
      <small> · {remote.kind === 'npc' ? `NPC type ${remote.type}` : 'player'} · {remote.ip}</small>
      <p>{remote.down ? 'This machine is down.' : remote.access ? 'Logged in as root.' : 'Not logged in.'}{remote.scanned && ' · Port scanned.'}</p>
      <button onClick={() => void bookmark()}>{remote.bookmarked ? 'Remove bookmark' : 'Bookmark'}</button>
      <button onClick={() => void act(internetApi.analyze(remote.ip), 'Analysis')}>Analyze</button>
      {remote.analysis && <p className="muted">CPU {remote.analysis.cpu_min}–{remote.analysis.cpu_max} · RAM {remote.analysis.ram_min}–{remote.analysis.ram_max}</p>}
      {!remote.access && !remote.down && <>
        <form className="compose" onSubmit={signIn}>
          <label>User<input value={user} onChange={e => setUser(e.target.value)} required /></label>
          <label>Password<input value={password} onChange={e => setPassword(e.target.value)} required /></label>
          <button>Log in</button>
        </form>
        <button onClick={() => void crack('bf')}>Crack password</button>
        <button onClick={() => void crack('scan')}>Port scan</button>
        <button onClick={() => void act(api.startProcess(16, remote.id, undefined, remote.kind), 'Exploit')}>Exploit FTP/SSH</button>
      </>}
    </article>}
    {remote?.access && <>
      <h3>Files</h3>
      {files.length ? files.map(f => <article className="forum-post" key={f.id}>
        <b>{f.name} v{f.version}</b><small> · {f.size} MB · type {f.type}</small>
        <button onClick={() => void act(internetApi.transfer(remote.ip, f.id, 'download'), 'Download')}>Download</button>
        <button onClick={() => void act(internetApi.removeFile(remote.ip, f.id), 'Delete')}>Delete</button>
      </article>) : <p className="muted">No files, or none visible.</p>}
      <h3>Log</h3>
      <pre className="wiki-content">{log || '(empty)'}</pre>
      <form className="compose" onSubmit={e => { e.preventDefault(); void act(internetApi.editLog(remote.ip, draft), 'Log edit') }}>
        <label>Replace the log with<textarea value={draft} onChange={e => setDraft(e.target.value)} /></label>
        <button>Rewrite log</button>
        <button type="button" onClick={() => void act(internetApi.editLog(remote.ip, ''), 'Log wipe')}>Wipe it</button>
      </form>
    </>}
    <h3>Recently visited</h3>
    {history.length ? history.map(h => <button key={h.ip} onClick={() => void visit(h.ip)}>{h.name || h.ip}</button>) : <p className="muted">Nowhere yet.</p>}
    <h3>Known machines</h3>
    <div className="table-wrap"><table><thead><tr><th>Name</th><th>IP</th><th>Type</th></tr></thead><tbody>
      {targets.map(t => <tr key={t.id}><td><button onClick={() => void visit(t.ip)}>{t.name || 'Unknown'}</button></td><td>{t.ip}</td><td>{t.type}</td></tr>)}
    </tbody></table></div>
  </>;
}
// Software. The list was already there as a raw table; what was missing is that every one
// of these verbs is a process, so the buttons queue work rather than doing it.
function SoftwareScreen() {
  const [files, setFiles] = useState<OwnedFile[]>([]);
  const [types, setTypes] = useState<{ id: number; name: string }[]>([]);
  const [name, setName] = useState('');
  const [type, setType] = useState('1');
  const [size, setSize] = useState('30');
  const [ram, setRam] = useState('50');
  const [message, setMessage] = useState('');
  const load = () => softwareApi.list().then(setFiles).catch(() => {});
  useEffect(() => { void load(); softwareApi.types().then(setTypes).catch(() => {}) }, []);
  const say = (err: unknown) => setMessage(err instanceof Error ? err.message : 'Request failed');
  const queue = async (action: number, id: number, label: string) => {
    try { await api.startProcess(action, undefined, id); setMessage(`${label} queued. Run the worker to advance it.`); load() } catch (err) { say(err) }
  };
  const research = async (id: number) => {
    try { const r = await researchApi.start(id); setMessage(`Research queued as task #${r.process_id}.`) } catch (err) { say(err) }
  };
  const write = async (event: FormEvent) => {
    event.preventDefault();
    try { await softwareApi.create(name, Number(type), Number(size), Number(ram)); setName(''); setMessage('Software written.'); load() } catch (err) { say(err) }
  };
  return <>
    <h2>Software</h2>
    <form className="compose" onSubmit={write}>
      <label>Name<input value={name} maxLength={25} onChange={e => setName(e.target.value)} required /></label>
      <label>Type<select value={type} onChange={e => setType(e.target.value)}>{types.map(t => <option key={t.id} value={t.id}>{t.name}</option>)}</select></label>
      <label>Size (MB)<input value={size} inputMode="numeric" onChange={e => setSize(e.target.value)} required /></label>
      <label>RAM<input value={ram} inputMode="numeric" onChange={e => setRam(e.target.value)} required /></label>
      <button>Write it</button>
      <span className="muted">{message}</span>
    </form>
    {files.map(f => <article className="forum-post" key={f.id}>
      <b>{f.name} v{f.version}</b>
      <small> · {f.size} MB · {f.ram} RAM · {f.running ? 'running' : 'not running'}{f.hidden && ` · hidden with v${f.hidden_with}`}</small>
      {f.running
        ? <button onClick={() => void queue(14, f.id, 'Uninstall')}>Uninstall</button>
        : <button onClick={() => void queue(13, f.id, 'Install')}>Install</button>}
      {f.hidden
        ? <button onClick={() => void queue(5, f.id, 'Seek')}>Seek</button>
        : <button onClick={() => void queue(4, f.id, 'Hide')}>Hide</button>}
      <button onClick={() => void research(f.id)}>Research</button>
      <button onClick={() => void queue(18, f.id, 'Park on external drive')}>Park</button>
    </article>)}
    {!files.length && <p className="muted">Nothing on disk yet. Write something, or go take it.</p>}
  </>;
}
// Hardware, with the shop hardwareItens.php kept in static arrays. A part replaces what is
// there, so the price is not an upgrade cost and buying down is allowed.
function HardwareScreen() {
  const [machines, setMachines] = useState<Machine[]>([]);
  const [parts, setParts] = useState<ShopPart[]>([]);
  const [accounts, setAccounts] = useState<Record<string, unknown>[]>([]);
  const [account, setAccount] = useState('');
  const [message, setMessage] = useState('');
  const load = () => hardwareApi.list().then(setMachines).catch(() => {});
  useEffect(() => { void load(); hardwareApi.shop().then(setParts).catch(() => {}); api.resource('/finances/accounts').then(setAccounts).catch(() => {}) }, []);
  const buy = async (part: ShopPart) => {
    if (!account) { setMessage('Pick the account that pays for it first.'); return }
    try { const r = await hardwareApi.buy(part.kind, part.id, Number(account)); setMessage(`${r.name} installed for $${r.paid}.`); load(); api.resource('/finances/accounts').then(setAccounts) } catch (err) { setMessage(err instanceof Error ? err.message : 'Request failed') }
  };
  const machine = machines[0];
  return <>
    <h2>Hardware</h2>
    {machine && <section className="cards">
      <article><b>{machine.cpu}</b><small>CPU</small></article>
      <article><b>{machine.hdd}</b><small>Disk (MB)</small></article>
      <article><b>{machine.ram}</b><small>RAM</small></article>
      <article><b>{machine.net}</b><small>Network</small></article>
    </section>}
    <label>Paying account<select value={account} onChange={e => setAccount(e.target.value)}>
      <option value="">Select an account</option>
      {accounts.map(a => <option key={String(a.account)} value={String(a.account)}>{String(a.account)} · ${String(a.money ?? a.cash ?? 0)}</option>)}
    </select></label>
    <p className="muted">{message}</p>
    {['cpu', 'hdd', 'ram', 'net'].map(kind => <div key={kind}>
      <h3>{kind.toUpperCase()}</h3>
      <div className="table-wrap"><table><thead><tr><th>Part</th><th>Power</th><th>Price</th><th></th></tr></thead><tbody>
        {parts.filter(p => p.kind === kind).map(p => <tr key={`${p.kind}${p.id}`}>
          <td>{p.name}</td><td>{p.power}</td><td>${p.price}</td>
          <td><button onClick={() => void buy(p)}>Buy</button></td>
        </tr>)}
      </tbody></table></div>
    </div>)}
  </>;
}
function DataTable({rows}:{rows:Record<string,unknown>[]}) { if(!rows.length) return <p className="muted">No records.</p>; const columns=Object.keys(rows[0]); return <div className="table-wrap"><table><thead><tr>{columns.map(c=><th key={c}>{c}</th>)}</tr></thead><tbody>{rows.map((row,i)=><tr key={String(row.id ?? i)}>{columns.map(c=><td key={c}>{String(row[c] ?? '')}</td>)}</tr>)}</tbody></table></div> }
